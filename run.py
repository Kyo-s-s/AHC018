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
    return seed, output

def progress(count):
    sys.stdout.write("\033[2K\033[G")
    print(f'{count}/{CASE}', end='', flush=True)


def main():
    scores = []
    count = 0
    with multiprocessing.Pool(max(1, multiprocessing.cpu_count()-2)) as pool:
        for seed, output in pool.imap_unordered(execute_case, range(CASE)):
            try:
                scores.append((int(output.split()[-1]), f'{seed:04}'))
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
    
if __name__ == '__main__':
    main()