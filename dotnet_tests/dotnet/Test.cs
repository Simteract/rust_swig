using System;
using System.Collections.Generic;
using Xunit;
using rust_swig_test_dotnet;

namespace dotnet
{
    public class UnitTest1
    {
        [Fact]
        public void Test1()
        {
            TestStaticClass.hello();
            TestStaticClass.print_number(123);
            Console.Out.WriteLine(TestStaticClass.add(1, 2));
            Console.Out.WriteLine(TestStaticClass.concat("Concatenated ", "String"));
            Console.Out.WriteLine(TestStaticClass.concat_str("Concatenated ", "str"));


            var obj = new TestClass();
            obj.print();
            obj.increment();
            obj.print();
            obj.add(3);
            obj.print();
            Console.Out.WriteLine(obj.get());
            
            TestStaticClass.test_obj_by_value(obj);

            var vec = new List<int>();
            vec.Add(1);
            vec.Add(2);

            TestStaticClass.print_vec_len(vec);
            var new_vec = TestStaticClass.get_vec();
            foreach (var e in new_vec)
            {
                Console.Out.WriteLine(e);
            }

            TestStaticClass.maybe_return_class(new Option<string>("asdf")).Value.print();
            Console.Out.WriteLine(TestStaticClass.maybe_add_one(new Option<int>()).IsSome);
        }
    }
}
