let r = 255;
let g = 0;
let b = 255;

let color = rgb8(r, g, b);

let color2 = rgb8(g, b, r);

let text = new_text("How does this even work???").color(color);
let text2 = new_text("Hello from Rhai!").color(color);

let text3 = new_text("Hello from iced!").color(color2);
let text4 = new_text("Hello aaaaaaaaaaaa!").color(color2);

let row = new_row([new_space_w(len_fill()), text4]);

let o_row = new_row([new_text("asdfasdf").color(rgb8(0, 255, 255))]);

let col = new_col([o_row, new_space_h(len_fill()), row, text2]);

let container_style = default_container_style();

container_style.background = rgb8(25, 25, 25).into_bg();

new_container(col).style(container_style)