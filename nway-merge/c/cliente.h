#pragma once

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>

typedef struct {
    int32_t cod;
    char nome[50];
    int64_t data_nascimento; // seconds since epoch
} Cliente;

void cliente_serialize(Cliente *cliente, FILE *out);

bool cliente_deserialize(Cliente *out, FILE *in);

