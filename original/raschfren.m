function raschfren;

%----------- global variables ----------------

global ed1 ed2 ed3 ed4 ed5 sl1
global u s c im xmin xmax
im=0;
%----------- text of function ----------------

n=100;

L1=str2num(get(ed1,'string'));
L2=str2num(get(ed2,'string'));
lyam=str2num(get(ed3,'string'));
kx=(2*(L1+L2)/(lyam.*L1.*L2))^0.5;

set(sl1,'min',str2num(get(ed4,'string')));
set(sl1,'max',str2num(get(ed5,'string')));
set(sl1,'value',str2num(get(ed5,'string')));

xmin=str2num(get(ed4,'string'));
xmax=str2num(get(ed5,'string'));
umin=kx*xmin;
umax=kx*xmax;
h=(umax-umin)/n;

if umax==-umin
    u=h:h:umax;
    c=fresnelc(u);
    s=fresnels(u);
    c=[-imrotate(c,180) fresnelc(0) c];
    s=[-imrotate(s,180) fresnels(0) s]; 
    u=umin:h:umax;
else
    u=umin:h:umax;
    c=fresnelc(u);
    s=fresnels(u);
end;

marker(3);