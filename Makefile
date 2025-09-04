japan:
	cargo run && cd z_orbit && convert -delay 2 $(ls *.ppm | sort -n) japan.gif

open:
	open -a "Google Chrome.app" z_orbit/japan.gif
