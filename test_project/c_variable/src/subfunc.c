#include "subfunc.h"

int subfunc(int x, int y)
{
  return x * y;
}

int subfunc2(int x, int y)
{
  global_value_b = y;
  return x + y;
 
}

