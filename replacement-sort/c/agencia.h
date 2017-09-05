#pragma once

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>

typedef struct agencia {
    int32_t cod;
    char nome[50];
    int32_t cod_gerente; // forein key references cod on funcionario
} Agencia;

void agencia_serialize(Agencia *agencia, FILE *out);

bool agencia_deserialize(FILE *in, Agencia *out);

