//! Abstract Syntax Tree

use syn::{Attribute, Expr, Ident, Item, ItemUse, Pat, PatType, Path, Stmt, Type};

use crate::syntax::Map;

/// The `#[app]` attribute
#[derive(Debug)]
#[non_exhaustive]
pub struct App {
    /// The arguments to the `#[app]` attribute
    pub args: AppArgs,

    /// The name of the `const` item on which the `#[app]` attribute has been placed
    pub name: Ident,

    /// The `#[init]` function
    pub init: Init,

    /// The `#[idle]` function
    pub idle: Option<Idle>,

    /// Monotonic clocks
    pub monotonics: Map<Monotonic>,

    /// Resources shared between tasks defined in `#[shared]`
    pub shared_resources: Map<SharedResource>,

    /// Task local resources defined in `#[local]`
    pub local_resources: Map<LocalResource>,

    /// User imports
    pub user_imports: Vec<ItemUse>,

    /// User code
    pub user_code: Vec<Item>,

    /// Hardware tasks: `#[task(binds = ..)]`s
    pub hardware_tasks: Map<HardwareTask>,

    /// Software tasks: `#[task]`
    pub software_tasks: Map<SoftwareTask>,
}

/// Interrupts used to dispatch software tasks
pub type Dispatchers = Map<Dispatcher>;

/// Interrupt that could be used to dispatch software tasks
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Dispatcher {
    /// Attributes that will apply to this interrupt handler
    pub attrs: Vec<Attribute>,
}

/// The arguments of the `#[app]` attribute
#[derive(Debug)]
pub struct AppArgs {
    /// Device
    pub device: Path,

    /// Peripherals
    pub peripherals: bool,

    /// Interrupts used to dispatch software tasks
    pub dispatchers: Dispatchers,
}

/// The `init`-ialization function
#[derive(Debug)]
#[non_exhaustive]
pub struct Init {
    /// `init` context metadata
    pub args: InitArgs,

    /// Attributes that will apply to this `init` function
    pub attrs: Vec<Attribute>,

    /// The name of the `#[init]` function
    pub name: Ident,

    /// The context argument
    pub context: Box<Pat>,

    /// The statements that make up this `init` function
    pub stmts: Vec<Stmt>,

    /// The name of the user provided shared resources struct
    pub user_shared_struct: Ident,

    /// The name of the user provided local resources struct
    pub user_local_struct: Ident,
}

/// `init` context metadata
#[derive(Debug)]
#[non_exhaustive]
pub struct InitArgs {
    /// Local resources that can be accessed from this context
    pub local_resources: LocalResources,
}

impl Default for InitArgs {
    fn default() -> Self {
        Self {
            local_resources: LocalResources::new(),
        }
    }
}

/// The `idle` context
#[derive(Debug)]
#[non_exhaustive]
pub struct Idle {
    /// `idle` context metadata
    pub args: IdleArgs,

    /// Attributes that will apply to this `idle` function
    pub attrs: Vec<Attribute>,

    /// The name of the `#[idle]` function
    pub name: Ident,

    /// The context argument
    pub context: Box<Pat>,

    /// The statements that make up this `idle` function
    pub stmts: Vec<Stmt>,
}

/// `idle` context metadata
#[derive(Debug)]
#[non_exhaustive]
pub struct IdleArgs {
    /// Local resources that can be accessed from this context
    pub local_resources: LocalResources,

    /// Shared resources that can be accessed from this context
    pub shared_resources: SharedResources,
}

impl Default for IdleArgs {
    fn default() -> Self {
        Self {
            local_resources: LocalResources::new(),
            shared_resources: SharedResources::new(),
        }
    }
}

/// Shared resource properties
#[derive(Debug)]
pub struct SharedResourceProperties {
    /// A lock free (exclusive resource)
    pub lock_free: bool,
}

/// A shared resource, defined in `#[shared]`
#[derive(Debug)]
#[non_exhaustive]
pub struct SharedResource {
    /// `#[cfg]` attributes like `#[cfg(debug_assertions)]`
    pub cfgs: Vec<Attribute>,

    /// `#[doc]` attributes like `/// this is a docstring`
    pub docs: Vec<Attribute>,

    /// Attributes that will apply to this resource
    pub attrs: Vec<Attribute>,

    /// The type of this resource
    pub ty: Box<Type>,

    /// Shared resource properties
    pub properties: SharedResourceProperties,
}

/// A local resource, defined in `#[local]`
#[derive(Debug)]
#[non_exhaustive]
pub struct LocalResource {
    /// `#[cfg]` attributes like `#[cfg(debug_assertions)]`
    pub cfgs: Vec<Attribute>,

    /// `#[doc]` attributes like `/// this is a docstring`
    pub docs: Vec<Attribute>,

    /// Attributes that will apply to this resource
    pub attrs: Vec<Attribute>,

    /// The type of this resource
    pub ty: Box<Type>,
}

/// Monotonic
#[derive(Debug)]
#[non_exhaustive]
pub struct Monotonic {
    /// `#[cfg]` attributes like `#[cfg(debug_assertions)]`
    pub cfgs: Vec<Attribute>,

    /// The identifier of the monotonic
    pub ident: Ident,

    /// The type of this monotonic
    pub ty: Box<Type>,

    /// Monotonic args
    pub args: MonotonicArgs,
}

/// Monotonic metadata
#[derive(Debug)]
#[non_exhaustive]
pub struct MonotonicArgs {
    /// The interrupt or exception that this monotonic is bound to
    pub binds: Ident,

    /// The priority of this monotonic
    pub priority: Option<u8>,

    /// If this is the default monotonic
    pub default: bool,
}

/// A software task
#[derive(Debug)]
#[non_exhaustive]
pub struct SoftwareTask {
    /// Software task metadata
    pub args: SoftwareTaskArgs,

    /// `#[cfg]` attributes like `#[cfg(debug_assertions)]`
    pub cfgs: Vec<Attribute>,

    /// Attributes that will apply to this interrupt handler
    pub attrs: Vec<Attribute>,

    /// The context argument
    pub context: Box<Pat>,

    /// The inputs of this software task
    pub inputs: Vec<PatType>,

    /// The statements that make up the task handler
    pub stmts: Vec<Stmt>,

    /// The task is declared externally
    pub is_extern: bool,

    /// If the task is marked as `async`
    pub is_async: bool,
}

/// Software task metadata
#[derive(Debug)]
#[non_exhaustive]
pub struct SoftwareTaskArgs {
    /// The task capacity: the maximum number of pending messages that can be queued
    pub capacity: u8,

    /// The priority of this task
    pub priority: u8,

    /// Local resources that can be accessed from this context
    pub local_resources: LocalResources,

    /// Shared resources that can be accessed from this context
    pub shared_resources: SharedResources,

    /// Only same priority tasks can spawn this task
    pub only_same_priority_spawn: bool,
}

impl Default for SoftwareTaskArgs {
    fn default() -> Self {
        Self {
            capacity: 1,
            priority: 1,
            local_resources: LocalResources::new(),
            shared_resources: SharedResources::new(),
            only_same_priority_spawn: false,
        }
    }
}

/// A hardware task
#[derive(Debug)]
#[non_exhaustive]
pub struct HardwareTask {
    /// Hardware task metadata
    pub args: HardwareTaskArgs,

    /// `#[cfg]` attributes like `#[cfg(debug_assertions)]`
    pub cfgs: Vec<Attribute>,

    /// Attributes that will apply to this interrupt handler
    pub attrs: Vec<Attribute>,

    /// The context argument
    pub context: Box<Pat>,

    /// The statements that make up the task handler
    pub stmts: Vec<Stmt>,

    /// The task is declared externally
    pub is_extern: bool,
}

/// Hardware task metadata
#[derive(Debug)]
#[non_exhaustive]
pub struct HardwareTaskArgs {
    /// The interrupt or exception that this task is bound to
    pub binds: Ident,

    /// The priority of this task
    pub priority: u8,

    /// Local resources that can be accessed from this context
    pub local_resources: LocalResources,

    /// Shared resources that can be accessed from this context
    pub shared_resources: SharedResources,
}

/// A `static mut` variable local to and owned by a context
#[derive(Debug)]
#[non_exhaustive]
pub struct Local {
    /// Attributes like `#[link_section]`
    pub attrs: Vec<Attribute>,

    /// `#[cfg]` attributes like `#[cfg(debug_assertions)]`
    pub cfgs: Vec<Attribute>,

    /// Type
    pub ty: Box<Type>,

    /// Initial value
    pub expr: Box<Expr>,
}

/// A wrapper of the 2 kinds of locals that tasks can have
#[derive(Debug)]
#[non_exhaustive]
pub enum TaskLocal {
    /// The local is declared externally (i.e. `#[local]` struct)
    External,
    /// The local is declared in the task
    Declared(Local),
}

/// Resource access
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Access {
    /// `[x]`, a mutable resource
    Exclusive,

    /// `[&x]`, a static non-mutable resource
    Shared,
}

impl Access {
    /// Is this enum in the `Exclusive` variant?
    pub fn is_exclusive(&self) -> bool {
        *self == Access::Exclusive
    }

    /// Is this enum in the `Shared` variant?
    pub fn is_shared(&self) -> bool {
        *self == Access::Shared
    }
}

/// Shared resource access list in task attribute
pub type SharedResources = Map<Access>;

/// Local resource access/declaration list in task attribute
pub type LocalResources = Map<TaskLocal>;
