import matplotlib.pyplot as plt
import subprocess
import json
import os

# Sample sizes to test (from 1 to 5000)
sample_sizes = [1, 5, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000]

output_file = "/home/gareth/dev/WikiStats/wiki-folder/stats/20250901.json"
data_dir = "/home/gareth/dev/WikiStats/wiki-folder/20250901/sqlite/"
wiki = "dewiki"
threads = 100

print(f"Output file: {output_file}")
print(f"Data directory: {data_dir}")
print(f"Wiki: {wiki}")


def run_sample_stats_for_sizes(size):
    """Run cargo sample-stats for multiple sample sizes"""
    # Configuration

    # Change to WikiGame directory
    os.chdir("/home/gareth/dev/WikiStats/WikiGame")

    print(f"Running sample-stats for sample size: {size}")
    cmd = [
        "cargo",
        "run",
        "--bin",
        "cli",
        "--release",
        "--",
        "sample-stats",
        "-s",
        str(size),
        "-t",
        str(threads),
        "-o",
        output_file,
        "-d",
        data_dir,
        "-w",
        wiki,
    ]

    print(f"Command: {' '.join(cmd)}")

    # Run the cargo command
    result = subprocess.run(cmd, capture_output=True, text=True)

    if result.returncode == 0:
        print(f"✓ Completed sample size {size}")
    else:
        print(f"✗ Failed sample size {size}")
        print(f"Error: {result.stderr}")
        return False


def load_bfs_sample_stats():
    """Load the bfs_sample_stats from the JSON file"""
    stats_file = "/home/gareth/dev/WikiStats/wiki-folder/stats/20250901.json"

    try:
        with open(stats_file, "r") as f:
            data = json.load(f)
            return data.get("bfs_sample_stats", {})
    except FileNotFoundError:
        print(f"Stats file not found: {stats_file}")
        return {}
    except json.JSONDecodeError as e:
        print(f"Error decoding JSON: {e}")
        return {}


def plot_histogram_for_sample_size(sample_size, histogram_data, output_dir="./"):
    """Plot histogram for a specific sample size and save as image"""
    if not histogram_data:
        print(f"No data found for sample size {sample_size}")
        return

    # Convert string keys to integers and prepare data
    avg_depth_histogram = {int(k): v for k, v in histogram_data.items()}
    depths = sorted(avg_depth_histogram.keys())
    means = [avg_depth_histogram[d]["avg_occurences"] for d in depths]
    stds = [avg_depth_histogram[d]["std_dev"] for d in depths]

    # Plot with log scale (because of huge range)
    plt.figure(figsize=(12, 6))
    plt.errorbar(
        depths, means, yerr=stds, fmt="o", capsize=5, markersize=4, linestyle="--"
    )

    plt.yscale("log")
    plt.xlabel("Depth")
    plt.ylabel("Average number of nodes (log scale)")
    plt.title(f"Average BFS Frontier Size per Depth (Sample Size: {sample_size})")
    plt.grid(True, which="both", linestyle="--", linewidth=0.5)

    # Save the plot
    filename = f"{output_dir}bfs_histogram_sample_{sample_size}.png"
    plt.savefig(filename, dpi=300, bbox_inches="tight")
    plt.close()  # Close the figure to free memory
    print(f"Saved plot: {filename}")


def main():
    """Main function to run sample stats and generate plots"""

    for sample_size in [1, 2]:
        success = run_sample_stats_for_sizes(sample_size)

        if not success:
            print("Failed to run sample-stats. Exiting.")
            return

        # Load the generated data
        print("\nStep 2: Loading bfs_sample_stats data...")
        bfs_data = load_bfs_sample_stats()

        if not bfs_data:
            print("No bfs_sample_stats found in the JSON file. Exiting.")
            return

        # Generate plots for each sample size
        print("\nStep 3: Generating plots for each sample size...")
        output_dir = "/home/gareth/dev/WikiStats/igraph/"

        for sample_size, histogram_data in bfs_data.items():
            if (
                isinstance(histogram_data, dict)
                and "avg_depth_histogram" in histogram_data
            ):
                plot_histogram_for_sample_size(
                    sample_size, histogram_data["avg_depth_histogram"], output_dir
                )

    print("\nAll plots generated successfully!")


# Run the main function
if __name__ == "__main__":
    main()
