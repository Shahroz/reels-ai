import glob

for fn in glob.glob("**/*.rs", recursive=True):
    with open(fn) as inp:
        contents = inp.read()
        if "query_as" in contents:
            print(fn)