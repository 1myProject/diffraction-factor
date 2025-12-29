function difrdemo;

%----------- global variables ----------------

global u sl1

xmin=get(sl1,'min');
xmax=get(sl1,'max');
h=(xmax-xmin)/100;
for i=1:100
    set(sl1,'value',xmin+h*i);
    marker(1);
    drawnow;
    pause(0.01);
end;
