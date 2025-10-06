import matplotlib.pyplot as plt
import numpy as np

# Data dictionary
data = {
    "31": {"avg_occurences": 4.472417927776714e-7, "std_dev": 2.1378467759617793e-22},
    "28": {"avg_occurences": 4.472417927776716e-7, "std_dev": 4.245792518944762e-22},
    "25": {"avg_occurences": 8.87774958663678e-7, "std_dev": 1.6460215226668724e-7},
    "22": {"avg_occurences": 2.1154536798383815e-6, "std_dev": 5.203921747737553e-7},
    "19": {"avg_occurences": 4.45676446502948e-6, "std_dev": 7.848005041153653e-7},
    "16": {"avg_occurences": 0.000010429678607575292, "std_dev": 1.3434173875956429e-6},
    "13": {"avg_occurences": 0.000023625547703480485, "std_dev": 4.028904544630401e-6},
    "10": {"avg_occurences": 0.00013040228952018563, "std_dev": 0.00009541966190862686},
    "7": {"avg_occurences": 0.01530514860279428, "std_dev": 0.020671767610876423},
    "4": {"avg_occurences": 0.2973624317788549, "std_dev": 0.11418715425288288},
    "1": {"avg_occurences": 0.00002348466653875552, "std_dev": 0.000055830855902315926},
    "32": {"avg_occurences": 4.472417927776712e-7, "std_dev": 0.0},
    "29": {"avg_occurences": 4.472417927776716e-7, "std_dev": 4.245792518944762e-22},
    "26": {"avg_occurences": 5.568160320082011e-7, "std_dev": 1.9283566386708017e-7},
    "23": {"avg_occurences": 1.4468271996357606e-6, "std_dev": 3.16969958674686e-7},
    "20": {"avg_occurences": 3.6696189097407937e-6, "std_dev": 3.179831000164381e-7},
    "17": {"avg_occurences": 8.155454091300834e-6, "std_dev": 1.1018975549129025e-6},
    "14": {"avg_occurences": 0.000018482267086537264, "std_dev": 2.64380415835635e-6},
    "11": {"avg_occurences": 0.00005863563524211658, "std_dev": 0.000024021204776723647},
    "8": {"avg_occurences": 0.0027134181929910966, "std_dev": 0.003438345830991981},
    "5": {"avg_occurences": 0.2613516474375058, "std_dev": 0.09960332195014585},
    "2": {"avg_occurences": 0.0027036146528934086, "std_dev": 0.009273321222794034},
    "30": {"avg_occurences": 4.472417927776716e-7, "std_dev": 4.2472479697738725e-22},
    "27": {"avg_occurences": 4.4947800174155995e-7, "std_dev": 3.1624770450311944e-8},
    "24": {"avg_occurences": 1.0957423923052925e-6, "std_dev": 2.2305832720480527e-7},
    "21": {"avg_occurences": 2.9741579219715074e-6, "std_dev": 4.223913541985771e-7},
    "18": {"avg_occurences": 6.196535038934631e-6, "std_dev": 1.0986270223178774e-6},
    "15": {"avg_occurences": 0.0000135983867094051, "std_dev": 2.4431102679027267e-6},
    "12": {"avg_occurences": 0.00003394565207182525, "std_dev": 8.827650526485778e-6},
    "9": {"avg_occurences": 0.0004969706077166204, "std_dev": 0.0006291438806812653},
    "6": {"avg_occurences": 0.08413840657588546, "std_dev": 0.08308835352457385},
    "3": {"avg_occurences": 0.08634758424581838, "std_dev": 0.08578809795695969}
}


# Extract and sort by key (1–32)
keys = sorted(data.keys(), key=lambda x: int(x))
print(keys)
values = [data[k]["avg_occurences"] for k in keys]

# Compute cumulative sum (CDF)
cumulative_values = np.cumsum(values)
# cumulative_values /= cumulative_values[-1]  # Normalize to 1

# Plot
plt.figure(figsize=(10, 6))
plt.plot(keys, cumulative_values, marker='o', linestyle='-', linewidth=2)
plt.title("Cumulative Histogram (Cumulative Distribution of avg_occurences)")
plt.xlabel("Key")
plt.ylabel("Cumulative Probability")
plt.grid(True, linestyle='--', alpha=0.6)
plt.tight_layout()
plt.show()
