# Memory Map

(Memory addresses are in hex, top bounds are exclusive.)
* 0 - 100: First page
* 100 - 200: Stack
* 200 - 2000: Base ROM file.
  * 200 - 206: ASCII `#!fun`, followed by LF.
  * 206 - 210: ASCII game title, padded with whitespace.
  * 210 - 300: To be determined.
  * 300 - 2000: Game code. Instruction pointer begins at 100.
* 2000 - 4000: 8 general-purpose ROM banks, each 1KiB long. 256 banks can exist in the ROM, and each map bank can switch between any of the ROM banks.
  * The high bank is special, in that it may be editable. If the bank has a write pin (or its filename ends with w, like game17w.bin) writes to this bank will be saved.
  * A writable bank may be swapped into other bank slots, but it may only be modified when in the high bank.
* 4000 - 4200: Control values.
  * 4000 - 4004: User input for players 1-4.
    * High nibble: Up, down, left, right.
    * Low nibble: A, B, select, start.
  * 4005: CPU control.
    * Bit 7: Halt the CPU until the start of the next frame.
    * Bit 6: Halt the CPU until the start of the next tile row.
    * Bit 5: Halt the CPU until the start of the next screen line.
    * Bit 4: Halt the CPU until the controller bytes are nonzero.
    * Bit 3: Halt the CPU until the start of the next tile.
    * Other bits are unused.
    * Only the highest bit will have an effect
  * 4006 - 4008:
    * A pointer to the background tilemap.
    * The tilemap is 289 bytes long, creating a 17x17 grid of tiles.
    * Each byte in the tilemap is an entry into the graphics table.
  * 4008 - 400A:
    * A pointer to the foreground tilemap.
  * 400A:
    * The top nibble determines horizontal scrolling.
    * The bottom nibble determines vertical scrolling.
    * Scrolling moves the tilemap left or up, respectively, to allow for the illusion of a larger world.
  * 400B - 400D:
    * A pointer to the graphics table.
    * Each tile is 8x8 pixels, using 2-bit color, for a total of 16 bytes per tile.
    * A full graphics table spans four banks.
  * 400D - 4010:
    * The color palette.
    * The highest three bits correspond to red.
    * The next highest three bits correspond to green.
    * The lowest two bits correspond to blue.
    * Color 00 is always black for the background, or transparent for the foreground or sprites.
    * Color 01 corresponds to 400D
    * 02 => 400E
    * 03 => 400F
  * 4010-4018:
    * Each byte corresponds with a map bank, and controls which ROM bank it contains.
  * 4019:
    * This byte controls stdio when running in an emulator. You can stream bytes to it to print.
* 4200 - 5000: Undecided
* 5000 - FFFF: Memory