
all: main.o
clean:
	rm -f *.o ray-tracer

main.o:
	g++ main.cpp -o ray-tracer -lglfw -lglew -framework opengl
