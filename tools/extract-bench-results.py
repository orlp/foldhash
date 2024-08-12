import json

from pathlib import Path

base_path = Path("target/criterion/")

print("bench,distr,hash,ns")
for path in base_path.glob("**/new/sample.json"):
    sample_path = path
    benchmark_path = path.parent / "benchmark.json"
    
    with benchmark_path.open() as benchmark_file, sample_path.open() as sample_file:
        name = json.load(benchmark_file)["function_id"]
        samples = json.load(sample_file)
        
        sample_times = sorted([t / n for t, n in zip(samples["times"], samples["iters"])])
        robust = sample_times[len(sample_times) // 10]
        print(",".join(name.split("-", 2)) + "," + str(robust))
