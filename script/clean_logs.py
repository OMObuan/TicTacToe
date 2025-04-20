import os
import shutil


def clean_logs():
    logs_dir = os.path.join(os.getcwd(), 'logs')
    print(logs_dir)
    if os.path.exists(logs_dir):
        for filename in os.listdir(logs_dir):
            file_path = os.path.join(logs_dir, filename)
            try:
                if os.path.isfile(file_path) or os.path.islink(file_path):
                    os.unlink(file_path)
                elif os.path.isdir(file_path):
                    shutil.rmtree(file_path)
            except Exception as e:
                print(f'Failed to delete {file_path}. Reason: {e}')


if __name__ == "__main__":
    clean_logs()
