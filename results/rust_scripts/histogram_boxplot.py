import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns

df = pd.read_json("/Users/rylan/blvflag/tool/logs/timings.jsonl", lines=True) # load timing log
df['timestamp'] = pd.to_datetime(df['timestamp']) # read just the time stamp and translate it to python readable

sns.histplot(df['duration_sec'], bins=20, kde=True)
plt.title("Runtime Distribution With --explain")
plt.xlabel("Duration (seconds)")
plt.ylabel("Count")
plt.show()

sns.boxplot(x=df['duration_sec'])
plt.title("Execution Time With --explain")
plt.ylabel("Duration (seconds)")
plt.show()
