import os
import sys
import numpy as np
import shutil

def generate_readme(folder_path):
    if not os.path.isdir(folder_path):
        print(f"Error: Folder '{folder_path}' not found.")
        return

    # Create or clean the readme_tmp directory
    tmp_dir = 'readme_tmp'
    if os.path.exists(tmp_dir):
        shutil.rmtree(tmp_dir)
    os.makedirs(os.path.join(tmp_dir, 'readme_pictures'))

    # Extract the name from the folder path
    name = os.path.basename(os.path.normpath(folder_path))

    # Start README content
    readme_content = f"# {name}\n\n"

    # Read description from description.txt
    description_file = os.path.join(folder_path, 'description.txt')
    if os.path.exists(description_file):
        with open(description_file, 'r') as f:
            description = f.read().strip()
        readme_content += f"{description}\n\n"
    else:
        readme_content += "No description found.\n\n"

    # Read and add sim.conf content
    sim_conf_file = os.path.join(folder_path, 'sim.conf')
    if os.path.exists(sim_conf_file):
        with open(sim_conf_file, 'r') as f:
            sim_conf_content = f.read()
        readme_content += "## Configuration\n\n"
        readme_content += f"```\n{sim_conf_content}\n```\n\n"

    # Find and copy statistic pictures
    readme_content += "## Statistics\n\n"
    statistic_pictures = [f for f in os.listdir(folder_path) if f.endswith('.png')]
    for pic in statistic_pictures:
        shutil.copy(os.path.join(folder_path, pic), os.path.join(tmp_dir, 'readme_pictures', pic))
        readme_content += f"![{pic}](readme_pictures/{pic})\n"
    readme_content += "\n"


    # Find, copy and select 5 state pictures uniformly
    readme_content += "## States\n\n"
    grid_states_path = os.path.join(folder_path, 'grid_states')
    state_pictures = []
    if os.path.isdir(grid_states_path):
        all_states = sorted([f for f in os.listdir(grid_states_path) if f.endswith('.png')])
        if len(all_states) > 5:
            indices = np.linspace(0, len(all_states) - 1, 5, dtype=int)
            state_pictures = [all_states[i] for i in indices]
        else:
            state_pictures = all_states

        for pic in state_pictures:
            shutil.copy(os.path.join(grid_states_path, pic), os.path.join(tmp_dir, 'readme_pictures', pic))
            readme_content += f"![{pic}](readme_pictures/{pic})\n"
    readme_content += "\n"


    # Write README.md
    readme_file = os.path.join(tmp_dir, 'README.md')
    with open(readme_file, 'w') as f:
        f.write(readme_content)

    print(f"README.md and pictures created in '{tmp_dir}'")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python generate_readme.py <folder_path>")
        sys.exit(1)

    folder_path = sys.argv[1]
    generate_readme(folder_path) 