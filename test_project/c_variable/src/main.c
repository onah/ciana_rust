#include <stdio.h>

#include "func.h"

int main()
{
   int a = 10;
   int b = a + 10;

   int ret = func(a, b);

   printf("%d\n", ret);

   return 0;
}


