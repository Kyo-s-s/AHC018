init:
	cd tools && cargo build --release && cd ..

compile:
	cd main && cargo build --release && cd ..

run:
	make compile && python run.py	