import hashlib
import base58

def CalcFileSha256(filname):
    ''' calculate file sha256 '''
    with open(filname, "rb") as f:
        sha256obj = hashlib.sha256()
        sha256obj.update(f.read())
        bvalue = sha256obj.digest()
        # hash_value = sha256obj.hexdigest()
        return bvalue

if __name__ == '__main__':
    import sys
    filepath = sys.argv[1]
    print("target file:", filepath)
    code_hash_local = bytes.decode(base58.b58encode(CalcFileSha256(filepath)))
    print("code hash:", code_hash_local)
