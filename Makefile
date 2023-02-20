init:
	cd tools && cargo build --release && cd ..

compile:
	cd main && cargo build --release && cd ..

run:
	make compile && python run.py	

run_new:
	make compile && python run_new.py

guess:
	main/target/release/guess_field < tools/in/0000.txt > tools/out/0000.txt