from ipykernel.kernelbase import Kernel
from jupyter_client.kernelspec import KernelSpecManager
import os

__version__ = '0.1.0'

class IRustKernel(Kernel):
    implementation = 'IRust'
    implementation_version = '1.0'
    language = 'rust'
    language_version = '1'
    language_info = {
        'name': 'IRust',
        'mimetype': 'text/plain',
        'file_extension': '.rs',
    }
    banner = "IRust"
    kernel_name = "irust"

    # Actual code
    repl = ""
    deps = []

    def do_execute(self, code, silent, store_history=True, user_expressions=None, allow_stdin=False):

        if code.startswith(':add'):
            code = code.removeprefix(':add').strip()
            self.deps.append(code)

        elif silent or code.endswith(';'):
            self.repl += code

        else:
            output = self.eval(self.repl + code, self.get_deps())
            stream_content = {'name': 'stdout', 'text': output}
            self.send_response(self.iopub_socket, 'stream', stream_content)

        return {'status': 'ok',
                # The base class increments the execution count
                'execution_count': self.execution_count,
                'payload': [],
                'user_expressions': {},
               }


    def get_deps(self):
        return (' ').join(self.deps)


    def eval(self, code, deps):
        import subprocess

        cmd = ["cmd","/c", os.path.join(self.get_kernel_location(),"re"),deps,code] if os.name == 'nt' else [os.path.join(self.get_kernel_location(),"re"),deps,code]
        output = subprocess.run(cmd, stdout=subprocess.PIPE)

        return output.stdout.decode("utf-8")
    
    
    def get_kernel_location(self):
        # Create a KernelSpecManager instance
        kernel_spec_manager = KernelSpecManager()
        # Get the kernel directory path
        kernel_dir =  os.path.join(str(kernel_spec_manager.user_kernel_dir),self.kernel_name)
        return kernel_dir



if __name__ == '__main__':
    from ipykernel.kernelapp import IPKernelApp
    IPKernelApp.launch_instance(kernel_class=IRustKernel)
