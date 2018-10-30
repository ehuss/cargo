import time
import os


def main():
    for n in range(0, 30):
        with open('fs_test', 'w') as file:
            file.write('a')
        st = os.stat('fs_test')
        print(st.st_mtime)
        time.sleep(0.1)


if __name__ == '__main__':
    main()
