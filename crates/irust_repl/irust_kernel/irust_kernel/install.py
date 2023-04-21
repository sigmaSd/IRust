import json
import os
import sys
import argparse
import subprocess
import shutil



from jupyter_client.kernelspec import KernelSpecManager
from IPython.utils.tempdir import TemporaryDirectory

kernel_json = {
  "argv": [sys.executable, "-m", "irust_kernel", "-f", "{connection_file}"],
  "display_name": "IRust",
  "language":"bash",
}

def install_my_kernel_spec(user=True, prefix=None, local_build=False):
    def get_cargo_target_dir():
        target = "target"
        if 'CARGO_TARGET_DIR' in os.environ:
            target = os.environ['CARGO_TARGET_DIR']
        return target


    with TemporaryDirectory() as td:
        cargo_target_dir = get_cargo_target_dir()
        os.chmod(td, 0o755) # Starts off as 700, not user readable
        with open(os.path.join(td, 'kernel.json'), 'w') as f:
            json.dump(kernel_json, f, sort_keys=True)

        if local_build:
            print('Building `Re` executable')
            try:
                subprocess.run(["cargo", "b", "--example", "re", "--target-dir", cargo_target_dir],check=True)
            except:
                print('--local-build needs to be used inside irust repo')
                exit(1)

            src = os.path.join(cargo_target_dir, "debug", "examples", "re")
            dst = os.path.join(td, "re")
            os.symlink(src, dst)

        else:
            print('Fetching `irust` repo and compiling `Re` executable')
            subprocess.run(["git", "clone","--depth","1", "https://github.com/sigmasd/irust"],cwd=td)
            irust_repl_dir = os.path.join(td,"irust", "crates", "irust_repl")
            subprocess.run(["cargo", "b", "--release", "--example", "re", "--target-dir",cargo_target_dir], cwd=irust_repl_dir)

            src = os.path.join(cargo_target_dir, "release", "examples", "re")
            dst = os.path.join(td, "re")
            shutil.copy2(src, dst)
            shutil.rmtree(os.path.join(td,"irust"))

        print('Installing IRust kernel spec')
        KernelSpecManager().install_kernel_spec(td, 'irust', user=user, prefix=prefix)
        print('done')

def _is_root():
    try:
        return os.geteuid() == 0
    except AttributeError:
        return False # assume not an admin on non-Unix platforms

def main(argv=None):
    parser = argparse.ArgumentParser(
        description='Install KernelSpec for IRust Kernel'
    )
    prefix_locations = parser.add_mutually_exclusive_group()

    prefix_locations.add_argument(
        '--user',
        help='Install KernelSpec in user\'s home directory',
        action='store_true'
    )
    prefix_locations.add_argument(
        '--sys-prefix',
        help='Install KernelSpec in sys.prefix. Useful in conda / virtualenv',
        action='store_true',
        dest='sys_prefix'
    )
    prefix_locations.add_argument(
        '--prefix',
        help='Install KernelSpec in this prefix',
        default=None
    )

    parser.add_argument('--local-build',
        help = "Build `Re` locally and copy it to the kernel location",
        default = False,
        action='store_true'
    )

    args = parser.parse_args(argv)

    user = False
    prefix = None
    if args.sys_prefix:
        prefix = sys.prefix
    elif args.prefix:
        prefix = args.prefix
    elif args.user or not _is_root():
        user = True

    install_my_kernel_spec(user=user, prefix=prefix, local_build=args.local_build)

if __name__ == '__main__':
    main()
