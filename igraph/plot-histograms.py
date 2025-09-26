import matplotlib.pyplot as plt
import subprocess
import json
import os
import numpy as np

# Sample sizes to test (from 1 to 5000)


output_file = "/home/gareth/dev/WikiStats/wiki-folder/stats/20250901.json"
data_dir = "/home/gareth/dev/WikiStats/wiki-folder/20250901/sqlite/"
wiki = "dewiki"
threads = 200

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
        "stats",
        "add-sample-stats",
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
        return True
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


def plot_histogram_for_sample_size(sample_size, histogram_data, output_dir="./", all_histograms=None):
    """Plot histogram for a specific sample size and save as image"""
    if not histogram_data:
        print(f"No data found for sample size {sample_size}")
        return

    # Convert string keys to integers and prepare data
    avg_depth_histogram = {int(k): v for k, v in histogram_data.items()}
    depths = sorted(avg_depth_histogram.keys())
    means = [avg_depth_histogram[d]["avg_occurences"] for d in depths]
    stds = [avg_depth_histogram[d]["std_dev"] for d in depths]

    cumsum_means = np.cumsum(means)

    # Plot
    plt.figure(figsize=(8, 5))
    plt.plot(depths, cumsum_means, marker='o', linestyle='-', color='blue', label='Cumulative occurrences')
    plt.fill_between(depths, cumsum_means - np.array(stds).cumsum(), 
                    cumsum_means + np.array(stds).cumsum(), color='blue', alpha=0.2, label='Cumulative ±1 std')


    # Plot with log scale (because of huge range)
    # plt.figure(figsize=(12, 6))
    # plt.plot(depths[0:15], means[0:15], marker="o", linestyle="--", color="blue", label="Mean")
    # plt.fill_between(
    #     depths[0:15],
    #     [m - s for m, s in zip(means[0:15], stds[0:15])],
    #     [m + s for m, s in zip(means[0:15], stds[0:15])],
    #     color="blue",
    #     alpha=0.2,
    #     label="Std Dev"
    # )
    # plt.legend()
    # plt.errorbar(
    #     depths, means, yerr=stds, fmt="o", capsize=5, markersize=4, linestyle="--"
    # )
    # Scatter plot using all_histograms (list of dicts: depth -> count)
    if all_histograms is not None:
        for idx, histogram in enumerate(all_histograms):
            depths = list(map(int, histogram.keys()))
            counts = list(histogram.values())

            # plt.scatter(depths, counts, label=f"Sample {idx+1}", alpha=1, s=1)
        
        
        all_depths_expanded = []
        # for histogram in all_histograms:
        #     if histogram is None or len(histogram.items()) == 0:
        #         continue
        #     expanded = []
        #     for depth, count in histogram.items():
        #         # print(f"Depth: {depth}, Count: {count}")
        #         num_points = max(1, int(count * 1e6))  # scale factor depends on your data
        #         expanded.extend([int(depth)] * num_points)  # repeat depth by its count
        #     all_depths_expanded.append(expanded)

        # vp = plt.violinplot(all_depths_expanded, showmeans=False, showmedians=True)
        # plt.legend(title="Samples", loc="upper right")

        # plt.xticks(range(1, len(all_depths_expanded) + 1), [f"Sample {i+1}" for i in range(len(all_depths_expanded))])
    # plt.yscale("log")

    # all_samples = []
    # for i in range(1, 15):
    #     sample1 = [h[str(i)] for h in all_histograms]
    #     all_samples.append(sample1)

    # # Create violin plot
    # plt.violinplot(all_samples, showmeans=True, showmedians=True)


    plt.xlabel("Depth")
    plt.ylabel("Average number of nodes (log scale)")
    plt.title(f"Average BFS Frontier Size per Depth (Sample Size: {sample_size})")
    # plt.grid(True, which="both", linestyle="--", linewidth=0.5)

    plt.show()
    # Save the plot
    # filename = f"{output_dir}bfs_histogram_sample_{sample_size}.png"
    # plt.savefig(filename, dpi=300, bbox_inches="tight")
    plt.close()  # Close the figure to free memory
    # print(f"Saved plot: {filename}")


def plot_all():
    """Main function to run sample stats and generate plots"""
    sample_sizes = [1, 5, 10, 25, 50, 100, 250, 300, 500, 750, 1000, 1500, 2500, 5000, 10_000]

    for sample_size in sample_sizes:
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
        output_dir = "/home/gareth/dev/WikiStats/igraph/hist/"

        histogram_data = bfs_data["dewiki"]
    
        plot_histogram_for_sample_size(
            sample_size, histogram_data["avg_depth_histogram"], output_dir
        )

    print("\nAll plots generated successfully!")


# Run the main function
if __name__ == "__main__":
    with open("./hist/depth_histogram_bfs.json", "r") as f:
        all_histograms = json.load(f) # is of type Vec<HashMap<u32, f64>>

    bfs_data = load_bfs_sample_stats()


    # Generate plots for each sample size
    print("\nStep 3: Generating plots for each sample size...")
    output_dir = "/home/gareth/dev/WikiStats/igraph/hist/"

    histogram_data = bfs_data["dewiki"]

    plot_histogram_for_sample_size(
        100, histogram_data["avg_depth_histogram"], output_dir, all_histograms
    )
