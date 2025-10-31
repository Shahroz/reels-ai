import glob

for fn in glob.glob("**/*.rs", recursive=True):
    with open(fn) as inp:
        content = inp.read()
        if "query_as" in content:
            print(fn)