import sys

def load_log_sp(filename):

    data = []
    with open(filename) as f:
        for line in f.readlines():
            tokens = line.split(" ")
            spidx = line.find("SP:")
            endidx = line.find(' ', spidx)
            data.append((line[0:4], line[spidx+3:endidx]))
    return data

if __name__ == "__main__":
    mylog = sys.argv[1]
    correctlog = sys.argv[2]
    mylog_sp = load_log_sp(mylog)
    correctlog_sp = load_log_sp(correctlog)
    for (i, ((nb1, sp1), (nb2, sp2))) in enumerate(zip(mylog_sp, correctlog_sp)):
        print('{} {} - {} vs {}'.format(
            nb1, nb2, sp1, sp2))
        if sp1.lower() != sp2.lower() or nb1.lower() != nb2.lower():
            break
