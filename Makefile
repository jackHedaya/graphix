render:
	cargo run && cd out && magick -delay 2 $$(ls *.ppm | sort -n) render.gif

open:
	open -a "Google Chrome.app" out/render.gif
