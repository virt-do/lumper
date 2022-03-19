#!/usr/bin/bash
apt install -y bison flex libelf-dev

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

LINUX_REPO=linux-cloud-hypervisor

if [ ! -d $LINUX_REPO ]
then
    git clone --depth 1 "https://github.com/cloud-hypervisor/linux.git" -b "ch-5.14" $LINUX_REPO
fi

pushd $LINUX_REPO
cp $SCRIPT_DIR/linux-config-x86_64 .config
make bzImage -j `nproc`
popd
