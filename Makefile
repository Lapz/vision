
leak_check:
	cargo instruments --release -t Leaks -p vision -- vision/fib.vis   
profile:
	cargo instruments --release -t 'CPU Profiler' -p vision -- vision/fib.vis