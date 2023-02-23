import subprocess
import pipes
import multiprocessing
import sys

CASE = 500
TL = 100

def execute_case(seed):
    input_file_path = f'tools/in/{seed:04}.txt'
    output_file_path = f'tools/out/{seed:04}.txt'
    pipe_file_path = f'tools/memo/pipefile_{seed:04}.txt' 
    with open(input_file_path) as fin:
        with open(output_file_path, 'w') as fout:
            with open(pipe_file_path, 'w') as fpipe:
                subprocess.run(['tools/target/release/tester', 'main/target/release/main'], stdin=fin, stdout=fout, stderr = fpipe, timeout=TL).stdout
            output = open(pipe_file_path).read()
            assert output 
    input_file = open(input_file_path).read()
    _, W, K, C = [int(x) for x in input_file.split()[0 : 4]]
    return seed, output, W, K, C

def progress(count):
    sys.stdout.write("\033[2K\033[G")
    print(f'{count}/{CASE}', end='', flush=True)


def main():
    scores = []
    count = 0
    scores_dict = {}
    with multiprocessing.Pool(max(1, multiprocessing.cpu_count()-2)) as pool:
        for seed, output, W, K, C in pool.imap_unordered(execute_case, range(CASE)):
            try:
                score = int(output.split()[-1])
                scores.append((score, f'{seed:04}'))
                if (W, K, C) not in scores_dict:
                    scores_dict[(W, K, C)] = (score, 1)
                else:
                    (su, cn) = scores_dict[(W, K, C)]
                    scores_dict[(W, K, C)] = (su + score, cn + 1)

            except ValueError:
                print(seed, "ValueError", flush = True)
                print(score, flush = True)
                exit()
            except IndexError:
                print(seed, "IndexError", flush = True)
                print(score, flush = True)
                exit()
            count += 1
            progress(count)

    print()
    scores.sort()
    total = sum([s[0] for s in scores])
    ave = total / CASE
    print(f'total: {total}')
    print(f'max: {scores[-1]}')
    print(f'ave: {ave}')
    print(f'min: {scores[0]}')

    print("--------------------------")
    print(f'{scores[-2]}')
    print(f'{scores[-3]}')
    print(f'{scores[-4]}')
    print(f'{scores[-5]}')
    print(f'{scores[-6]}')
    print("--------------------------")

    dict_W = {}
    dict_K = {}
    dict_C = {}
    for (W, K, C), (score, count) in scores_dict.items():
        if W not in dict_W:
            dict_W[W] = (score, count)
        else:
            (su, cn) = dict_W[W]
            dict_W[W] = (su + score, cn + count)

        if K not in dict_K:
            dict_K[K] = (score, count)
        else:
            (su, cn) = dict_K[K]
            dict_K[K] = (su + score, cn + count)

        if C not in dict_C:
            dict_C[C] = (score, count)
        else:
            (su, cn) = dict_C[C]
            dict_C[C] = (su + score, cn + count)

    list_W = []
    list_K = []
    list_C = []
    
    for W, (score, count) in dict_W.items():
        list_W.append((W, score / count))

    for K, (score, count) in dict_K.items():
        list_K.append((K, score / count))

    for C, (score, count) in dict_C.items():
        list_C.append((C, score / count))

    list_W.sort()
    list_K.sort()
    list_C.sort()

    print("--------------------------")
    for W, ave in list_W:
        print(f'W: {W:3}, average: {ave}')
    print("--------------------------")
    for K, ave in list_K:
        print(f'K: {K:3}, average: {ave}')
    print("--------------------------")
    for C, ave in list_C:
        print(f'C: {C:3}, average: {ave}')
    print("--------------------------")

if __name__ == '__main__':
    main()