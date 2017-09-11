#include "cliente.h"

typedef struct node {
    Cliente val;
    struct node *left;
    struct node *right;
} Node;

static Node *leafify(Cliente val)
{
    Node *node = malloc(sizeof(Node));

    if (!node) abort();

    node->val = val;
    node->left = node->right = NULL;

    return node;
}

Node *build_tree(Cliente *array, size_t n)
{
    Node **leafs = malloc(sizeof(Node *) * n);

    for (size_t i = 0; i < n; ++i) {
        leafs[i] = leafify(array[i]);
    }


    for (size_t i = 1; i < n; ++i) {
        Node *left  = leafs[i - 1],
             *right = leafs[i];
    }
}

int main(void)
{
    return 0;
}
