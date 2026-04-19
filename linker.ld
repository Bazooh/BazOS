ENTRY(_start)

SECTIONS
{
    . = 0xffff800000000000;

    /* Code */
    .text : ALIGN(4096)
    {
        *(.text .text.*)
    }

    /* Read-only data */
    .rodata : ALIGN(4096)
    {
        *(.rodata .rodata.*)
    }

    /* Initialized data */
    .data : ALIGN(4096)
    {
        *(.data .data.*)
    }

    /* Uninitialized data */
    .bss : ALIGN(4096)
    {
        *(COMMON)
        *(.bss .bss.*)
    }

    /* Drop useless sections */
    /DISCARD/ :
    {
        *(.comment)
        *(.eh_frame)
        *(.note*)
    }
}