import subprocess
import os

script_dir = "/Users/rylan/blvflag/results/test_scripts"

'''
# diff metric code
def modify_script(filepath, j):
    with open(filepath, 'r') as file:
        lines = file.readlines()

    
    for idx, line in enumerate(lines): # modify first line we find from StackOverflow
        if line.strip():
            lines[idx] = f'# MODIFIED VERSION {j}\n'
            break

    with open(filepath, 'w') as file:
        file.writelines(lines)

# loop to run + modify and then re-run each test script
for i in range(5):  
    for j in range(1, 21): 
        script_path = os.path.join(script_dir, f"test{j}.py")

        subprocess.run(["cargo", "run", script_path, "--diff"])

        # call and modify with stack overflow boilder
        modify_script(script_path, j)

        subprocess.run(["cargo", "run", script_path, "--diff"])    # second run

'''


'''
for i in range(5): # a simple script to run all test files for metric generation
    for j in range(1, 21):
        subprocess.run(["cargo", "run", f"/Users/rylan/blvflag/results/test_scripts/test{j}.py", "--diff"])

for i in range(5): # a simple script to run all test files for metric generation
    for j in range(9, 21):
        subprocess.run(["cargo", "run", f"/Users/rylan/blvflag/results/test_scripts/test{j}.py", "--explain"])

for i in range(5): # a simple script to run all test files for metric generation
    for j in range(1, 21):
        subprocess.run(["cargo", "run", f"/Users/rylan/blvflag/results/test_scripts/test{j}.py"])
'''




