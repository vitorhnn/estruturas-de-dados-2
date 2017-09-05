#include "agencia.h"
#include <endian.h>

#define FIX_ENDIANNESS(x) x = be32toh(x)

void agencia_serialize(Agencia *agencia, FILE *out)
{
    int32_t codbe = htobe32(agencia->cod);
    fwrite(&codbe, sizeof(int32_t), 1, out);
    fwrite(agencia->nome, 1, sizeof(agencia->nome), out);

    int32_t cod_gerentebe = htobe32(agencia->cod_gerente);
    fwrite(&cod_gerentebe, sizeof(int32_t), 1, out);
}

bool agencia_deserialize(FILE *in, Agencia *out)
{
    size_t read = fread(&out->cod, sizeof(int32_t), 1, in);

    if (read < 1) {
        return false;
    }

    read = fread(out->nome, 1, sizeof(out->nome), in);

    if (read < sizeof(out->nome)) {
        return false;
    }

    read = fread(&out->cod_gerente, sizeof(int32_t), 1, in);

    if (read < 1) {
        return false;
    }

    FIX_ENDIANNESS(out->cod);
    FIX_ENDIANNESS(out->cod_gerente);

    return true;
}

