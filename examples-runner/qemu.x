MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 32K
  RAM :   ORIGIN = 0x20000000, LENGTH = 32K
  # RAM only runner
  # FLASH : ORIGIN = 0x20000000,     LENGTH = 22K
  # RAM :   ORIGIN = 0x20000000+22K, LENGTH = 10K
}

/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
/* NOTE Do NOT modify `_stack_start` unless you know what you are doing */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
