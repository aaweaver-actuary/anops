from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from typing import ClassVar as _ClassVar, Optional as _Optional

DESCRIPTOR: _descriptor.FileDescriptor

class PredictRequest(_message.Message):
    __slots__ = ("input_data",)
    INPUT_DATA_FIELD_NUMBER: _ClassVar[int]
    input_data: str
    def __init__(self, input_data: _Optional[str] = ...) -> None: ...

class PredictResponse(_message.Message):
    __slots__ = ("output_data",)
    OUTPUT_DATA_FIELD_NUMBER: _ClassVar[int]
    output_data: str
    def __init__(self, output_data: _Optional[str] = ...) -> None: ...
