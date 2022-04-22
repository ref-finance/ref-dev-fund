#/bin/bash
FILE=${1:-releases/session_vault_release.wasm}
PYTHON_IMAGE=python
pushd $(dirname $0) > /dev/null
# echo $(pwd)

docker ps -a | grep python_hashcode || docker create \
    --mount type=bind,source=${PWD},target=/host \
    --cap-add=SYS_PTRACE --security-opt seccomp=unconfined \
    --name=python_hashcode \
    -w /host \
    -it \
    ${PYTHON_IMAGE} \
    /bin/bash
docker ps | grep python_hashcode || docker start python_hashcode 
docker exec python_hashcode pip3 install base58
docker exec python_hashcode python3 codehash.py ${FILE}

popd > /dev/null
# echo $(pwd)
