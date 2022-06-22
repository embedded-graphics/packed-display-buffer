# Packed display buffer draw target for embedded-graphics

Prototype state right now, but useful for displays like SH1106 and SSD1306 which use a single byte
to encode 8 pixel on/off values.

## TODO/ideas/wants

- [x] Add support for `fill_contiguous` as well as solid fills
- [ ] Add support for active area tracking so partial updates are possible

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
