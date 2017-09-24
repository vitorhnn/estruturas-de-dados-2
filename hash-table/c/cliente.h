#pragma once

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>

typedef struct {
    uint32_t cod;
    char nome[100];
} Cliente;

void cliente_serialize(Cliente *cliente, FILE *out);

bool cliente_deserialize(Cliente *out, FILE *in);

