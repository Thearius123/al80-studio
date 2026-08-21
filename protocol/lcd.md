# AL80 LCD Protocol Notes

Status: reverse-engineered / experimental

## Physical image format

- width: 96 px
- height: 160 px
- RGB565
- big-endian pixel bytes
- row-major
- full image payload: 30,720 bytes

## UART

Current tested baud rate:

`921600`

Approximate full-frame wire minimum at 10 UART bits per byte:

`333.333 ms`

## Frame format

Observed frame structure:

`A5 5A TYPE LENGTH_BE16 CRC16 PAYLOAD`

CRC:

CRC16/MODBUS over TYPE + LENGTH fields.

## Known frames

GO_HOME:

`A5 5A 0B 00 00 02 00`

TOGGLE_PIC:

`A5 5A 0D 00 00 03 E0`

GUI_EVENT 1:

`A5 5A 10 00 01 C5 B1 01`

## Safety

Do not send protocol type `0x16`.

Persistent behavior of image/media commands such as ADD_PIC is not
fully characterized.

Application UI must label persistent-media behavior experimental until
verified.
