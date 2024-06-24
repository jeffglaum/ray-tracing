
all: main.o
clean:
	rm -f *.o ray-tracer

main.o:
	g++ main.cpp -g -o ray-tracer -lglfw -lglew -framework opengl
