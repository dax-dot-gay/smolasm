# smolasm
Little configurable assembler to support (mostly) arbitrary assembly languages

**Summary:**

`smolasm` is essentially a tool for defining a validated input assembly language and translating that into raw bytecode, which hardcoded formatters then transform into valid machine code. Input formats can be arbitrarily defined in the tool's configuration, and new output formats can be added to [src/formats](./src/formats). New output formats may be requested in an issue or (ideally) contributed via PR.

**Supported Output Formats:**

| Format Name   | Description   |
| :---:         | ---           |
| `ritarch`     | The RIT CS Dept's ARCH library module format |
| ...           | ...           |

**CLI Usage:**

The CLI is documented completely with `smolasm help`, but some examples are as follows:

```sh
smolasm input.asm analyze # Analyzes the memory layout of input.asm
smolasm input.asm asm # Assembles input.asm into input.obj
smolasm input.asm asm -o beans.obj # Assembles input.asm into beans.obj
```

**Configuration:**

`smolasm` is configured with [KDL](https://kdl.dev/). The default config file path is `config.kdl`, but others can be specified with the `-c/--config` flag in the CLI.

<details>
<summary>Detailed configuration format</summary>

```kdl
system {
    name "<the name of the ISA/assembly language>"
    format "<the output format name, ie 'ritarch'>"
    hardware {
        address_size    8 // Size of memory addresses in bits
        word_size       8 // Size of a unit of memory in bits
    }
}

// Fields are predefined sections that appear in instructions.
// Instructions are composed entirely of fields.
field "<field name, ie 'opcode'>" {
    type "enum" // This field is an "enum" field, meaning it has a set number of possible values
    bits 4      // How many bits long this field is
    
    // Enum fields can have up to 2^bits variants
    variant 0b0000 { // The variant discriminator (what this enum will translate into in bytecode)
        name "hlt" // The primary mnemonic of this variant
        alias "halt" // Any number of aliases can be specified as well
        alias "stop"
        alias ...
    }

    variant 0b0001 {
        name "dmp" // Names and aliases should be unique
    }

    ...
}

field "imm" {
    type "raw" // This field is "raw", meaning it represents any value that can fit in its bit-width.
    bits 8
}

...

// Individual instructions should be defined with an opcode enum.
// `instruction` configs, instead, specify different instruction formats.
// The assembler will raise an error if an instruction doesn't match exactly one format.
instruction "<instruction type>" { // The name of this instruction format. Must be unique
    // Each field in a format is described with a `field` entry.
    // Each has the following arguments:
    //  - value: The name of the field to use here, optionally with `=a|b|c` to constrain allowed enum variants (not allowed for raw fields)
    //  - in: The index that this field appears in in the assembly code
    //  - out: The index that this field appears in in the output machine code
    //  - default (optional): A raw numerical value that, if specified, makes this field optional. Only applicable to the final fields of an instruction (ie non-defaults cannot follow defaults)
    field value="opcode" in=0 out=0
    field value="ra" in=1 out=1 default=0
}

// Any number of instructions can be specified
instruction "<other instruction>" {
    field value="opcode" in=0 out=0
    field value="ra" in=1 out=1
    field value="imm" in=2 out=2
}
```

</details>
<br>

**Unified Assembly Format:**

All assembly programs for `smolasm` follow the same format, though their instructions may vary. Code is separated into `.data` and `.text` blocks, describing raw data and executable code, respectively. The format for these blocks is as follows:

```
// Data block
// <name> should be unique and contain no spaces
// <start> is either a memory address or "auto", which places it at the next available address.
// <length> is either a length (in # of addresses) or "auto", which sizes the block based on the contained data.
// Individual items within a data block are not aligned to word boundaries, but the block as a whole is.
.data <name> <start|auto>..<length|auto>
    """
    multiline string
    """
    "single line string"

    // Hex values
    0x10

    // Binary values
    0b0110

    // Unsigned integers (no floats or negatives cuz eh)
    500

// Text block
// Header format is identical to above.
// Each instruction is automatically aligned to word boundaries
// Immediate/raw fields can also be replaced with @<label>+/-offset, which will return the absolute address of that label (if it exists)
.text <name> <start|auto>..<length|auto>
    add 1 1
    jmp @other_block
    ...
```

More in-depth code samples can be found in [Accept.asm](./Accept.asm) and [input.asm](./input.asm).

