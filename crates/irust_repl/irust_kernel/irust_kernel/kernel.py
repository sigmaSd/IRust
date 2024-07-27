"""IRust repl jupyter kernel """

__version__ = '0.5.0'

from ipykernel.kernelbase import Kernel
from jupyter_client.kernelspec import KernelSpecManager
import os
import json
import subprocess

class IRustKernel(Kernel):
    implementation = 'IRust'
    implementation_version = '1.0'
    language = 'rust'
    language_version = '1'
    language_info = {
        'name': 'IRust',
        'mimetype': 'text/x-rust',
        'file_extension': '.rs',
    }
    banner = "IRust"
    kernel_name = "irust"

    def __init__(self, **kwargs):
        cmd = ["cmd","/c", os.path.join(self.get_kernel_location(),"re")] if os.name == 'nt' else [os.path.join(self._get_kernel_location(),"re"),]
        self.re = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        super().__init__(**kwargs)


    def do_execute(self, code, silent, store_history=True, user_expressions=None, allow_stdin=False):
        # Send the first JSON object to the process's standard input
        code_object = {"Execute": {"code": code}}
        json_code = json.dumps(code_object)
        self.re.stdin.write(json_code.encode("utf-8"))
        self.re.stdin.write(b"\n")
        self.re.stdin.flush()

        # Read the first JSON object from the process's standard output
        json_result = self.re.stdout.readline().decode("utf-8")
        action_type = json.loads(json_result)

        if "Eval" in action_type:
            action = action_type["Eval"]
            self.send_response(self.iopub_socket, 'display_data', {
                'metadata': {},
                'data': {
                    action["mime_type"]: action["value"]
                }
            })

        return {'status': 'ok',
                'execution_count': self.execution_count,
                'payload': [],
                'user_expressions': {},
               }


    def _get_kernel_location(self):
        kernel_spec_manager = KernelSpecManager()
        kernel_dir =  os.path.join(str(kernel_spec_manager.user_kernel_dir),self.kernel_name)
        return kernel_dir



if __name__ == '__main__':
    from ipykernel.kernelapp import IPKernelApp
    IPKernelApp.launch_instance(kernel_class=IRustKernel)
