import sys

def load_right_sp(filename):

    data = []
    with open(filename) as f:
        for line in f.readlines():
            tokens = line.split(" ")
            spidx = line.find("SP:")
            endidx = line.find(' ', spidx)
            data.append((tokens[0], line[spidx+3:endidx]))
    return data

if __name__ == "__main__":
    print(load_right_sp('correct.log'))
