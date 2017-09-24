#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#include "cliente.h"

typedef struct {
    FILE *table;
    FILE *entries;
    size_t size;
} FileHashTable;

FileHashTable table_open(FILE *table, FILE *entries, size_t size);

FileHashTable table_new(FILE *table, FILE *entries, size_t size);

void table_insert(FileHashTable *self, uint64_t key, Cliente val);

