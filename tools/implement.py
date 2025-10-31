import glob
import os


for fn in glob.glob("crates/agentloop/**/*.rs", recursive=True):
    print(fn)
    with open(fn) as inp:
        contents = inp.read()

    if "todo" in contents.lower() or "stub" in contents.lower() or "placeholder" in contents.lower():
        print(fn)
        instruction = f"Please implement the file {fn} fully (as a recursive task). The working folder is crates/agentloop/ for interactions with LLMs use crates/llm/. for tools implementation browsing and searching use crates/api_clients/. Use the additional crates if needed."
        os.system(f"""zen do --instruction "{instruction}" --patterns 'crates/**'""")