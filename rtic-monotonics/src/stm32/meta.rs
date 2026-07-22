//! Compile-time lookups into the `stm32-metapac` metadata.
//!
//! The chip is selected by the chip feature enabled on the `stm32-metapac`
//! dependency, so all lookups resolve to constants for that chip. A missing
//! peripheral or RCC entry is a compile-time panic.

use stm32_metapac::metadata::{
    ir::{BitOffset, BlockItemInner},
    Peripheral, PeripheralRccRegister, METADATA,
};

pub const NVIC_PRIO_BITS: u8 = match METADATA.nvic_priority_bits {
    Some(bits) => bits,
    None => panic!("stm32-metapac metadata does not provide `nvic_priority_bits`"),
};

// Errata ES0005 2.1.11: delay after an RCC peripheral clock enabling.
const DSB_AFTER_ENABLE: bool = streq(METADATA.family, "STM32F2");

/// RCC enable/reset bits for a timer, resolved at compile time.
pub struct TimerRcc {
    enable: RccBit,
    reset: Option<RccBit>,
}

impl TimerRcc {
    pub const fn lookup(timer: &str) -> Self {
        let rcc = match &find_peripheral(timer).rcc {
            Some(rcc) => rcc,
            None => panic!("timer has no RCC metadata in stm32-metapac"),
        };
        Self {
            enable: match &rcc.enable {
                Some(enable) => RccBit::lookup(enable),
                None => panic!("timer has no RCC enable metadata in stm32-metapac"),
            },
            reset: match &rcc.reset {
                Some(reset) => Some(RccBit::lookup(reset)),
                None => None,
            },
        }
    }

    pub fn enable(&self) {
        self.enable.set(true);
        if DSB_AFTER_ENABLE {
            cortex_m::asm::dsb();
        }
    }

    pub fn reset(&self) {
        if let Some(reset) = &self.reset {
            reset.set(true);
            reset.set(false);
        }
    }
}

/// A single bit in an RCC register.
#[derive(Clone, Copy)]
struct RccBit {
    addr: u32,
    mask: u32,
}

impl RccBit {
    /// Resolves an RCC (register, field) name pair to (address, mask) by
    /// walking the RCC register block IR.
    const fn lookup(bit: &PeripheralRccRegister) -> Self {
        let rcc = find_peripheral("RCC");
        let regs = match &rcc.registers {
            Some(regs) => regs,
            None => panic!("RCC has no registers in stm32-metapac metadata"),
        };

        let mut b = 0;
        while b < regs.ir.blocks.len() {
            let block = &regs.ir.blocks[b];
            if !streq(block.name, regs.block) {
                b += 1;
                continue;
            }

            let mut i = 0;
            while i < block.items.len() {
                let item = &block.items[i];
                if !streq(item.name, bit.register) {
                    i += 1;
                    continue;
                }
                assert!(
                    item.array.is_none(),
                    "RCC register arrays are not supported"
                );
                let fieldset = match &item.inner {
                    BlockItemInner::Register(reg) => match reg.fieldset {
                        Some(fieldset) => fieldset,
                        None => panic!("RCC register has no fieldset"),
                    },
                    BlockItemInner::Block(_) => panic!("expected an RCC register, found a block"),
                };
                return Self {
                    addr: rcc.address as u32 + item.byte_offset,
                    mask: 1 << field_bit(regs.ir.fieldsets, fieldset, bit.field),
                };
            }
            panic!("RCC register not found in stm32-metapac metadata");
        }
        panic!("RCC register block not found in stm32-metapac metadata");
    }

    /// Non-atomic read-modify-write, matching the plain `modify` the
    /// build-script generated code used before.
    fn set(&self, on: bool) {
        let reg = self.addr as *mut u32;
        unsafe {
            let val = reg.read_volatile();
            reg.write_volatile(if on {
                val | self.mask
            } else {
                val & !self.mask
            });
        }
    }
}

const fn field_bit(
    fieldsets: &[stm32_metapac::metadata::ir::FieldSet],
    fieldset: &str,
    field: &str,
) -> u32 {
    let mut f = 0;
    while f < fieldsets.len() {
        if !streq(fieldsets[f].name, fieldset) {
            f += 1;
            continue;
        }
        let fields = fieldsets[f].fields;
        let mut i = 0;
        while i < fields.len() {
            if streq(fields[i].name, field) {
                match &fields[i].bit_offset {
                    BitOffset::Regular(offset) => return offset.offset,
                    BitOffset::Cursed(_) => panic!("cursed bit offset in RCC field"),
                }
            }
            i += 1;
        }
        panic!("RCC field not found in stm32-metapac metadata");
    }
    panic!("RCC fieldset not found in stm32-metapac metadata");
}

const fn find_peripheral(name: &str) -> &'static Peripheral {
    let mut i = 0;
    while i < METADATA.peripherals.len() {
        if streq(METADATA.peripherals[i].name, name) {
            return &METADATA.peripherals[i];
        }
        i += 1;
    }
    panic!("peripheral not found in stm32-metapac metadata");
}

/// Case-insensitive ASCII compare; IR names are lowercase while RCC
/// register/field names are uppercase.
const fn streq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if !a[i].eq_ignore_ascii_case(&b[i]) {
            return false;
        }
        i += 1;
    }
    true
}
