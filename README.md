# Packed display buffer draw target for embedded-graphics

Prototype state right now, but useful for displays like SH1106 and SSD1306 which use a single byte
to encode 8 pixel on/off values.

## TODO/ideas/wants

- [x] Add support for `fill_contiguous` as well as solid fills
- [x] Add support for active area tracking so partial updates are possible
- [ ] Add support for 0/90/180/270º rotations
- [ ] Support bit depths > 1 and <= 8 for e.g. 2bpp 3 colour epaper displays

## Test cases TODO

- [ ] Display size
  - [ ] W and H ARE multiples of 8
  - [ ] W is not a multiple of 8
  - [ ] H is not a multiple of 8
  - [ ] Neither are a multiple of 8
- [ ] Blank display
  - [ ] Perfect fill where area = display size
  - [ ] Zero sized at origin
  - [ ] Zero sized at center
  - [ ] Rectangle larger than display area on all sides
  - [ ] Rectangle half off each edge
  - [ ] Rectangle half off each corner
  - [ ] Two overlapping rectangles
  - [ ] Zero sized rectangle
  - [ ] Single pixel high rectangle
  - [ ] Single pixel wide rectangle
- [ ] Display with existing content
  - [ ] Filled rectangle over background, `BinaryColor::On`
  - [ ] Filled rectangle over background, `BinaryColor::Off`
  - [ ] Zero sized rectangle
