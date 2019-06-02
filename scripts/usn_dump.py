import os
import re
import ujson
import pytsk3
import logging
import argparse
import pyrustyusn

VALID_DEBUG_LEVELS = ["ERROR", "WARN", "INFO", "DEBUG"]
__VERSION__ = "0.2.0"

RE_WIN_LOGICAL = re.compile(r"\\\\[.]\\[a-zA-Z]:", re.I)
RE_WIN_VOLSHAD = re.compile(r"\\\\\?\\GLOBALROOT\\Device\\HarddiskVolumeShadowCopy\d{1,3}", re.I)


def set_debug_level(debug_level):
    if debug_level in VALID_DEBUG_LEVELS:
        logging.basicConfig(
            level=getattr(logging, debug_level)
        )
    else:
        raise (Exception("{} is not a valid debug level.".format(debug_level)))


def get_arguments():
    usage = "Parse USN records from a file, extracted unallocated, or volume handle" \
            " (file system or volume shadow).".format(__VERSION__)

    arguments = argparse.ArgumentParser(
        description=usage,
        formatter_class=argparse.RawDescriptionHelpFormatter
    )
    arguments.add_argument(
        "-s", "--source",
        dest="source",
        action="store",
        required=True,
        help="The USN Journal, directory, or a logical volume (logical volume handle: \\\\.\\C: "
             "or \\\\?\\GLOBALROOT\\Device\\HarddiskVolumeShadowCopy6). If source is a directory, "
             "it will recurse through the folders looking for files that end with $J."
    )
    arguments.add_argument(
        "-v", "--is_volume",
        dest="is_volume",
        action="store_true",
        required=False,
        default=None,
        help="The source is a volume handle."
    )
    arguments.add_argument(
        "--debug",
        dest="debug",
        action="store",
        default="ERROR",
        choices=VALID_DEBUG_LEVELS,
        help="Debug level [default=ERROR]"
    )

    return arguments


def enumerate_volume_handle(source):
    """ Enumerate if the source is a volume handle
    """
    if RE_WIN_LOGICAL.match(source):
        logging.info("Logical matched drive letter: {}".format(source))
        return True
    elif RE_WIN_VOLSHAD.match(source):
        logging.info("Volume shadow matched: {}".format(source))
        return True
    
    return False


def process_dir(directory, options):
    for subdir, dirs, files in os.walk(directory):
        for file in files:
            file_path = subdir + os.sep + file
            if file_path.lower().endswith("$j"):
                process_file(
                    file_path,
                    options
                )


def process_file(file_location, options):
    with open(file_location, "rb") as fh:
        process_io_handle(
            file_location, fh, options
        )


def process_io_handle(location, io_handle, options):
    logging.info("Processing: {}".format(location))
    parser = pyrustyusn.PyUsnParser(location, io_handle)
    for record in parser.records():
        print(ujson.dumps(record))


def process_logical(options):
    """Process the source as a logical volume.

    :param options: tool options
    :return:
    """
    tsk_img = pytsk3.Img_Info(
        options.source
    )
    tsk_fs = pytsk3.FS_Info(
        tsk_img
    )

    fs_location = u"/$Extend/$UsnJrnl"

    # Look for the USN Journal for this file system
    try:
        tsk_file = tsk_fs.open(fs_location)
    except Exception as error:
        raise error

    for attr in tsk_file:
        if attr.info.type == pytsk3.TSK_FS_ATTR_TYPE_NTFS_DATA:
            if attr.info.name == b"$J":
                tsk_file_io = TskFileIo.from_tsk_file_and_attribute(
                    fs_location, attr, tsk_file
                )

                source_description = u"{}".format(
                    os.sep.join([options.source, fs_location[1:].replace("/", os.sep)])
                )
                process_io_handle(
                    source_description, tsk_file_io, options
                )


def main():
    arguments = get_arguments()
    options = arguments.parse_args()

    set_debug_level(
        options.debug
    )

    logical_flag = options.is_volume
    if logical_flag is None:
        logical_flag = enumerate_volume_handle(
            options.source
        )

    if logical_flag:
        process_logical(options)
    else:
        if os.path.isfile(options.source):
            process_file(options.source, options)
        elif os.path.isdir(options.source):
            process_dir(options.source, options)
        else:
            raise(Exception("Source is not a directory or a file."))


class FileInfo(object):
    def __init__(self, fullname, attribute):
        self.fullname = fullname
        self.filename = attribute.info.fs_file.name.name
        self.id = attribute.info.id
        self.type = attribute.info.type
        self.size = attribute.info.size
        self.attribute_name = attribute.info.name


class TskFileIo(object):
    """Class that implements a file-like object using pytsk3."""
    def __init__(self, tsk_file, tsk_file_info):
        """Initializes the file-like object.
        Args:
            tsk_file (File): the tsk File object
            tsk_file_info (FileInfo): The file info representing the tsk_file.
                                      This contains the attribute to read from.
        """
        self.tsk_file = tsk_file
        self.tsk_file_info = tsk_file_info
        self._current_offset = 0

    @staticmethod
    def from_tsk_file_and_attribute(full_path, attribute, tsk_file):
        """Get a TSK File like IO object.

        :param full_path: The fs path to the file
        :param attribute: The TSK Attribute to use for reading
        :param tsk_file: The TSK File object
        :return:
        """
        file_info = FileInfo(
            full_path, attribute
        )

        return TskFileIo(
            tsk_file, file_info
        )

    def read(self, size=None):
        """Implement the read functionality.
        Args:
            size: The size to read.
        Returns:
            bytes
        """

        if self._current_offset < 0:
            raise IOError(u'Invalid current offset value less than zero.')

        if self._current_offset >= self.tsk_file_info.size:
            return b''

        if size is None or self._current_offset + size > self.tsk_file_info.size:
            size = self.tsk_file_info.size - self._current_offset

        data = self.tsk_file.read_random(
            self._current_offset,
            size,
            self.tsk_file_info.type,
            self.tsk_file_info.id
        )

        self._current_offset += len(data)

        return data

    def seek(self, offset, whence=os.SEEK_SET):
        """Seeks an offset within the file-like object.
        Args:
            offset: The offset to seek.
            whence: Optional value that indicates whether offset is an absolute or relative position within the file.
        """
        if whence == os.SEEK_CUR:
            offset += self._current_offset
        elif whence == os.SEEK_END:
            offset += self.tsk_file_info.size
        elif whence != os.SEEK_SET:
            raise IOError(u'Unsupported whence.')

        if offset < 0:
            raise IOError(u'Invalid offset value less than zero.')

        self._current_offset = offset

        # return new position
        return self._current_offset

    def get_offset(self):
        """Get the current offset.
        Returns:
            file offset
        """
        return self._current_offset

    def get_size(self):
        """Get the file's size.
        Returns:
            file size
        """
        return self.tsk_file_info.size

    def tell(self):
        """Alias for get_offset()
        Returns:
            file offset
        """
        return self.get_offset()


if __name__ == "__main__":
    main()
