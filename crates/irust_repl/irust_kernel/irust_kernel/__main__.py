from ipykernel.kernelapp import IPKernelApp
from .kernel import IRustKernel
IPKernelApp.launch_instance(kernel_class=IRustKernel)
