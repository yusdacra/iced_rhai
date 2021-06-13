let r = 0.to_u8();
let g = 0.to_u8();
let b = 255.to_u8();

let color = new_color(r, g, b);
let text = new_text("Hello from iced!").color(color);

text.into_element()