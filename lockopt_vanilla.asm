
target/thumbv7m-none-eabi/release/examples/lockopt:     file format elf32-littlearm


Disassembly of section .text:

000000f0 <GPIOA>:
  f0:	b580      	push	{r7, lr}
  f2:	20a0      	movs	r0, #160	; 0xa0
  f4:	f380 8811 	msr	BASEPRI, r0
  f8:	f240 0000 	movw	r0, #0
  fc:	f2c2 0000 	movt	r0, #8192	; 0x2000
 100:	6801      	ldr	r1, [r0, #0]
 102:	3101      	adds	r1, #1
 104:	6001      	str	r1, [r0, #0]
 106:	f24e 2000 	movw	r0, #57856	; 0xe200
 10a:	2102      	movs	r1, #2
 10c:	f2ce 0000 	movt	r0, #57344	; 0xe000
 110:	6001      	str	r1, [r0, #0]
 112:	2104      	movs	r1, #4
 114:	6001      	str	r1, [r0, #0]
 116:	2126      	movs	r1, #38	; 0x26
 118:	20e0      	movs	r0, #224	; 0xe0
 11a:	f380 8811 	msr	BASEPRI, r0
 11e:	f2c0 0102 	movt	r1, #2
 122:	2018      	movs	r0, #24
 124:	f000 f892 	bl	24c <__syscall>
 128:	2000      	movs	r0, #0
 12a:	f380 8811 	msr	BASEPRI, r0
 12e:	bd80      	pop	{r7, pc}

00000130 <GPIOB>:
 130:	21a0      	movs	r1, #160	; 0xa0
 132:	f3ef 8011 	mrs	r0, BASEPRI
 136:	f381 8811 	msr	BASEPRI, r1
 13a:	f240 0100 	movw	r1, #0
 13e:	f2c2 0100 	movt	r1, #8192	; 0x2000
 142:	680a      	ldr	r2, [r1, #0]
 144:	3201      	adds	r2, #1
 146:	600a      	str	r2, [r1, #0]
 148:	21c0      	movs	r1, #192	; 0xc0
 14a:	f381 8811 	msr	BASEPRI, r1
 14e:	f380 8811 	msr	BASEPRI, r0
 152:	4770      	bx	lr

00000154 <GPIOC>:
 154:	f240 0100 	movw	r1, #0
 158:	f3ef 8011 	mrs	r0, BASEPRI
 15c:	f2c2 0100 	movt	r1, #8192	; 0x2000
 160:	680a      	ldr	r2, [r1, #0]
 162:	3202      	adds	r2, #2
 164:	600a      	str	r2, [r1, #0]
 166:	f380 8811 	msr	BASEPRI, r0
 16a:	4770      	bx	lr

0000016c <main>:
 16c:	f24e 1000 	movw	r0, #57600	; 0xe100
 170:	f24e 4201 	movw	r2, #58369	; 0xe401
 174:	f2ce 0000 	movt	r0, #57344	; 0xe000
 178:	21e0      	movs	r1, #224	; 0xe0
 17a:	b672      	cpsid	i
 17c:	f880 1300 	strb.w	r1, [r0, #768]	; 0x300
 180:	2101      	movs	r1, #1
 182:	f2ce 0200 	movt	r2, #57344	; 0xe000
 186:	23c0      	movs	r3, #192	; 0xc0
 188:	6001      	str	r1, [r0, #0]
 18a:	7013      	strb	r3, [r2, #0]
 18c:	2302      	movs	r3, #2
 18e:	6003      	str	r3, [r0, #0]
 190:	23a0      	movs	r3, #160	; 0xa0
 192:	7053      	strb	r3, [r2, #1]
 194:	2204      	movs	r2, #4
 196:	6002      	str	r2, [r0, #0]
 198:	f64e 5210 	movw	r2, #60688	; 0xed10
 19c:	f2ce 0200 	movt	r2, #57344	; 0xe000
 1a0:	6813      	ldr	r3, [r2, #0]
 1a2:	f043 0302 	orr.w	r3, r3, #2
 1a6:	6013      	str	r3, [r2, #0]
 1a8:	f8c0 1100 	str.w	r1, [r0, #256]	; 0x100
 1ac:	b662      	cpsie	i
 1ae:	bf30      	wfi
 1b0:	e7fd      	b.n	1ae <main+0x42>

000001b2 <Reset>:
 1b2:	f000 f84a 	bl	24a <DefaultPreInit>
 1b6:	f240 0004 	movw	r0, #4
 1ba:	f240 0100 	movw	r1, #0
 1be:	f2c2 0000 	movt	r0, #8192	; 0x2000
 1c2:	f2c2 0100 	movt	r1, #8192	; 0x2000
 1c6:	4281      	cmp	r1, r0
 1c8:	d214      	bcs.n	1f4 <Reset+0x42>
 1ca:	f240 0100 	movw	r1, #0
 1ce:	2200      	movs	r2, #0
 1d0:	f2c2 0100 	movt	r1, #8192	; 0x2000
 1d4:	f841 2b04 	str.w	r2, [r1], #4
 1d8:	4281      	cmp	r1, r0
 1da:	bf3c      	itt	cc
 1dc:	f841 2b04 	strcc.w	r2, [r1], #4
 1e0:	4281      	cmpcc	r1, r0
 1e2:	d207      	bcs.n	1f4 <Reset+0x42>
 1e4:	f841 2b04 	str.w	r2, [r1], #4
 1e8:	4281      	cmp	r1, r0
 1ea:	d203      	bcs.n	1f4 <Reset+0x42>
 1ec:	f841 2b04 	str.w	r2, [r1], #4
 1f0:	4281      	cmp	r1, r0
 1f2:	d3ef      	bcc.n	1d4 <Reset+0x22>
 1f4:	f240 0000 	movw	r0, #0
 1f8:	f240 0100 	movw	r1, #0
 1fc:	f2c2 0000 	movt	r0, #8192	; 0x2000
 200:	f2c2 0100 	movt	r1, #8192	; 0x2000
 204:	4281      	cmp	r1, r0
 206:	d21c      	bcs.n	242 <Reset+0x90>
 208:	f240 2168 	movw	r1, #616	; 0x268
 20c:	f240 0200 	movw	r2, #0
 210:	f2c0 0100 	movt	r1, #0
 214:	f2c2 0200 	movt	r2, #8192	; 0x2000
 218:	680b      	ldr	r3, [r1, #0]
 21a:	f842 3b04 	str.w	r3, [r2], #4
 21e:	4282      	cmp	r2, r0
 220:	d20f      	bcs.n	242 <Reset+0x90>
 222:	684b      	ldr	r3, [r1, #4]
 224:	f842 3b04 	str.w	r3, [r2], #4
 228:	4282      	cmp	r2, r0
 22a:	bf3e      	ittt	cc
 22c:	688b      	ldrcc	r3, [r1, #8]
 22e:	f842 3b04 	strcc.w	r3, [r2], #4
 232:	4282      	cmpcc	r2, r0
 234:	d205      	bcs.n	242 <Reset+0x90>
 236:	68cb      	ldr	r3, [r1, #12]
 238:	3110      	adds	r1, #16
 23a:	f842 3b04 	str.w	r3, [r2], #4
 23e:	4282      	cmp	r2, r0
 240:	d3ea      	bcc.n	218 <Reset+0x66>
 242:	f7ff ff93 	bl	16c <main>
 246:	defe      	udf	#254	; 0xfe

00000248 <DefaultHandler_>:
 248:	Address 0x0000000000000248 is out of bounds.


00000249 <ADC0_SEQUENCE_0>:
 249:	Address 0x0000000000000249 is out of bounds.


0000024a <DefaultPreInit>:
 24a:	Address 0x000000000000024a is out of bounds.


0000024b <__pre_init>:
 24b:	Address 0x000000000000024b is out of bounds.


0000024c <__syscall>:
 24c:	beab      	bkpt	0x00ab
 24e:	4770      	bx	lr

00000250 <HardFaultTrampoline>:
 250:	4670      	mov	r0, lr
 252:	2104      	movs	r1, #4
 254:	4208      	tst	r0, r1
 256:	d102      	bne.n	25e <HardFaultTrampoline+0xe>
 258:	f3ef 8008 	mrs	r0, MSP
 25c:	e002      	b.n	264 <HardFault_>
 25e:	f3ef 8009 	mrs	r0, PSP
 262:	e7ff      	b.n	264 <HardFault_>

00000264 <HardFault_>:
 264:	Address 0x0000000000000264 is out of bounds.


00000265 <HardFault>:
 265:	Address 0x0000000000000265 is out of bounds.

