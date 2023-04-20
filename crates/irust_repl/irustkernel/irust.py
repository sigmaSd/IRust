from ipykernel.kernelbase import Kernel

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
        import os

        cmd = ["cmd","/c","re",deps,code] if os.name == 'nt' else ["re",deps,code]
        output = subprocess.run(cmd, stdout=subprocess.PIPE)

        return output.stdout.decode("utf-8")


if __name__ == '__main__':
    from ipykernel.kernelapp import IPKernelApp
    IPKernelApp.launch_instance(kernel_class=IRustKernel)
