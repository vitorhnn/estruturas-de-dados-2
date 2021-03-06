#include "hash_table.h"
#include "cliente.h"

int main(void)
{
    FileHashTable table = table_new(fopen("hash", "w+"), fopen("data", "w+"), 7);
    Cliente c = {
        0,
        "Motocicleberson"
    };

    table_insert(&table, c.cod, c);
    c.cod += 7;
    table_insert(&table, c.cod, c);
    c.cod += 7;
    table_insert(&table, c.cod, c);

    table_delete(&table, c.cod - 7);

    return 0;
}
