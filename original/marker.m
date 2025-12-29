function marker(p);

%----------- global variables ----------------

global axes1 axes2 axes3 axes2b axes3b axes4
global txt1 sl1 
global ed1 ed2 ed3 ed4 ed5 ed6
global u s c im xmin xmax
global mr31 mr32 mr33 mr34 mr35 mrf1 mrf2 ekr yekr
%----------- text of function ----------------
eps=1e2;
L1=str2num(get(ed1,'string'));
L2=str2num(get(ed2,'string'));
lyam=str2num(get(ed3,'string'));
kx=(2*(L1+L2)/(lyam.*L1.*L2))^0.5;
F=exp(j*pi/4)/2^0.5*(0.5+c-j*(0.5+s));

switch p
case 1
    x=get(sl1,'value');
    set(ed6,'string',num2str(x));
case 2
    x=str2num(get(ed6,'string'));
    set(sl1,'value',x);
case 3
    u0=zeros(1,length(u));
    axes(axes1);
    hold off;
    %график
    plot3(c,u,s,'LineWidth',2);
    hold on;
    %проекции
    plot3(u0+1,u,s,':g','LineWidth',2); 
    plot3(c,u,u0-1,':g','LineWidth',2);
    plot3(c,u0+floor(min(u)),s,':g','LineWidth',2);
    %подписи к осям
    xlabel('C(u)');
    ylabel('u');
    zlabel('S(u)');
    grid on;
    axis ij;
    
    %дифракционный множитель: модуль и фаза
        
    axes(axes2b);
    hold off;
    plot(u,abs(F),'b');
    grid on;
    
    axes(axes2);
    hold off;
    plot(u,abs(F),'b');
    grid on;
    
    axes(axes3b);
    hold off;
    plot(u,unwrap(angle(F)),'b');
    grid on;
    
    axes(axes3);
    hold off;
    plot(u,unwrap(angle(F)),'b');
    grid on;
    
    %зоны Френеля
    axes(axes4);
    hold off;
    t=0:pi/180:2*pi;
    b=((lyam*L1*L2)/(L1+L2))^0.5;
    xc=cos(t);
    yc=sin(t);
    for n=round(2*xmax^2/b^2):-1:1
        if mod(n,2)==0
            fill(n^0.5*b*xc,n^0.5*b*yc,'b');
        else
            fill(n^0.5*b*xc,n^0.5*b*yc,'r');
        end;
        hold on;
    end;
    %yekr=n^0.5*b;
    yekr=(round(2*xmax^2/b^2))^0.5*b;
    axis([xmin,xmax,-xmax,xmax])
    hold off;
end;

if im==0 
    x=get(sl1,'value');
    ut=x*kx;
    ct=fresnelc(ut);
    st=fresnels(ut);
    Ft=exp(j*pi/4)/2^0.5*(0.5+ct-j*(0.5+st));
    axes(axes1);hold on;
    %маркер 3D
    mr31=plot3(ct,ut,st,'or');
    mr32=plot3(1,ut,st,'or');
    mr33=plot3(ct,ut,-1,'or');
    mr34=plot3(ct,floor(min(u)),st,'or');
    mr35=plot3([ct 0],[ut 0],[st 0],'k','LineWidth',2);
    %маркеp F модyль и фаза
    axes(axes2);hold on;
    mrf1=plot(ut,abs(Ft),'or','EraseMode','background','MarkerFaceColor',[1 0 0]);
    axes(axes3);hold on;
    mrf2=plot(ut,unwrap(angle(Ft)),'or','EraseMode','background','MarkerFaceColor',[1 0 0]);
    %экран
    axes(axes4);hold on;
    ekr=fill([x x xmax xmax],[-yekr yekr yekr -yekr],'k','EraseMode','background');
    im=1;
else
    ut=x*kx;
    ct=fresnelc(ut);
    st=fresnels(ut);
    Ft=exp(j*pi/4)/2^0.5*(0.5+ct-j*(0.5+st));
    
    set(mr31,'XData',ct,'YData',ut,'ZData',st);
    set(mr32,'XData',1,'YData',ut,'ZData',st);
    set(mr33,'XData',ct,'YData',ut,'ZData',-1);
    set(mr34,'XData',ct,'YData',floor(min(u)),'ZData',st);
    set(mr35,'XData',[ct 0],'YData',[ut 0],'ZData',[st 0]);
    
    %маркеp F модyль и фаза
    set(mrf1,'XData',ut,'YData',abs(Ft));
    set(mrf2,'XData',ut,'YData',unwrap(angle(Ft)));
    %экран
    set(ekr,'XData',[x x xmax xmax],'YData',[-yekr yekr yekr -yekr]);
    
end;

%вывод расчётных значений
set(txt1,'string',...
        [' u=',num2str(fix(ut*eps)/eps),...
         ' s ( u )=',num2str(fix(st*eps)/eps),...
         ' c ( u )=',num2str(fix(ct*eps)/eps),...
         ' F ( u )=',num2str(fix(Ft*eps)/eps)]);