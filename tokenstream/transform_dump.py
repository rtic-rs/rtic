
with open('tokenstream/tokenstream.txt', 'r') as f:
    lines = f.readlines()

result = ""

line = lines[0]

tabs = 0
newline = False

for i in range(len(line)):
    
    if line[i] == ",":
        result += line[i] + "\n" + "  "*tabs
        newline = True
    elif (line[i] == "{") or (line[i] == "["):
        tabs += 1
        result += line[i] + "\n" + "  "*tabs
        newline = True
    elif (line[i] == "}") or (line[i] == "]"):
        tabs -= 1
        result += line[i]
    else:
        if newline and (line[i] == " "):
            newline = False
            continue
        result += line[i]

with open('tokenstream/formated_tokenstream.txt', 'w') as f:
    f.write(result)
