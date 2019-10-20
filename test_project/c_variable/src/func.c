#include "func.h"
#include "subfunc.h"

int func(int num1, int num2)
{
  int ret = num1 + subfunc(num1, num2);
  ret = ret + 2;
  global_value_b = ret;
  return ret;
}


