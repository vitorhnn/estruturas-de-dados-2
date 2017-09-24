#include "cliente.h"
#include <endian.h>

void cliente_serialize(Cliente *cliente, FILE *out)
{
    uint32_t codbe = htobe32(cliente->cod);
    fwrite(&codbe, sizeof(uint32_t), 1, out);
    fwrite(cliente->nome, 1, sizeof(cliente->nome), out);
}

bool cliente_deserialize(Cliente *out, FILE *in)
{
    size_t read = fread (&out->cod, sizeof(uint32_t), 1, in);

    if (read < 1) {
        return false;
    }

    read = fread(out->nome, 1, sizeof(out->nome), in);

    if (read < sizeof(out->nome)) {
        return false;
    }

    out->cod = be32toh(out->cod);

    return true;
}
