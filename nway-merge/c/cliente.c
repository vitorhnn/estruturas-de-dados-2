#include "cliente.h"
#include <endian.h>

void cliente_serialize(Cliente *cliente, FILE *out)
{
    int32_t codbe = htobe32(cliente->cod);
    fwrite(&codbe, sizeof(int32_t), 1, out);
    fwrite(cliente->nome, 1, sizeof(cliente->nome), out);

    int64_t databe = htobe64(cliente->data_nascimento);
    fwrite(&databe, sizeof(int64_t), 1, out);
}

bool cliente_deserialize(Cliente *out, FILE *in)
{
    size_t read = fread (&out->cod, sizeof(int32_t), 1, in);

    if (read < 1) {
        return false;
    }

    read = fread(out->nome, 1, sizeof(out->nome), in);

    if (read < sizeof(out->nome)) {
        return false;
    }

    read = fread(&out->data_nascimento, sizeof(int64_t), 1, in);

    if (read < 1) {
        return false;
    }

    out->cod = be32toh(out->cod);
    out->data_nascimento = be64toh(out->data_nascimento);

    return true;
}
